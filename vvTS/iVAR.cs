using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000031 RID: 49
	[HandlerCategory("vvIndicators"), HandlerDecimals(2), HandlerName("iVAR")]
	public class iVAR : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x060001BD RID: 445 RVA: 0x000086DC File Offset: 0x000068DC
		public IList<double> Execute(ISecurity sec)
		{
			return this.Context.GetData("iVAR", new string[]
			{
				this.Length.ToString(),
				this.Smooth.ToString(),
				sec.get_CacheName()
			}, () => iVAR.GenIVAR(sec, this.Length, this.Smooth));
		}

		// Token: 0x060001BC RID: 444 RVA: 0x00008524 File Offset: 0x00006724
		public static IList<double> GenIVAR(ISecurity _sec, int _length, int _smooth)
		{
			int count = _sec.get_Bars().Count;
			IList<double> highPrices = _sec.get_HighPrices();
			IList<double> lowPrices = _sec.get_LowPrices();
			double[] array = new double[count];
			for (int i = 0; i < count; i++)
			{
				if (i < 65)
				{
					array[i] = 0.5;
				}
				else
				{
					double num = 0.0;
					double num2 = 0.0;
					double num3 = 0.0;
					double num4 = 0.0;
					for (int j = 0; j <= _length; j++)
					{
						int num5 = Convert.ToInt32(Math.Pow(2.0, (double)(_length - j)));
						double num6 = Math.Pow(2.0, (double)j);
						double num7 = 0.0;
						int num8 = 0;
						while ((double)num8 < num6)
						{
							double num9 = Indicators.Highest(highPrices, i - num5 * num8, num5);
							double num10 = Indicators.Lowest(lowPrices, i - num5 * num8, num5);
							num7 += num9 - num10;
							num8++;
						}
						double num11 = (double)(_length - j) * Math.Log(2.0);
						double num12 = Math.Log(num7);
						num += num11;
						num2 += num12;
						num3 += num11 * num11;
						num4 += num11 * num12;
					}
					array[i] = -(num * num2 - (double)(_length + 1) * num4) / (num * num - (double)(_length + 1) * num3);
				}
			}
			IList<double> result = array;
			if (_smooth > 0)
			{
				result = JMA.GenJMA(array, _smooth, 0);
			}
			return result;
		}

		// Token: 0x17000095 RID: 149
		public IContext Context
		{
			// Token: 0x060001BE RID: 446 RVA: 0x00008751 File Offset: 0x00006951
			get;
			// Token: 0x060001BF RID: 447 RVA: 0x00008759 File Offset: 0x00006959
			set;
		}

		// Token: 0x17000093 RID: 147
		[HandlerParameter(true, "5", Min = "2", Max = "20", Step = "1")]
		public int Length
		{
			// Token: 0x060001B8 RID: 440 RVA: 0x00008501 File Offset: 0x00006701
			get;
			// Token: 0x060001B9 RID: 441 RVA: 0x00008509 File Offset: 0x00006709
			set;
		}

		// Token: 0x17000094 RID: 148
		[HandlerParameter(true, "3", Min = "2", Max = "15", Step = "1")]
		public int Smooth
		{
			// Token: 0x060001BA RID: 442 RVA: 0x00008512 File Offset: 0x00006712
			get;
			// Token: 0x060001BB RID: 443 RVA: 0x0000851A File Offset: 0x0000671A
			set;
		}
	}
}
