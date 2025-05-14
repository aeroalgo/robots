using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000192 RID: 402
	[HandlerCategory("vvAverages"), HandlerName("OMA")]
	public class OMA : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000CBE RID: 3262 RVA: 0x0003738C File Offset: 0x0003558C
		public IList<double> Execute(IList<double> price)
		{
			return this.Context.GetData("OMA", new string[]
			{
				this.Length.ToString(),
				this.Speed.ToString(),
				this.ERadaptive.ToString(),
				price.GetHashCode().ToString()
			}, () => OMA.GenOMA(price, this.Length, this.Speed, this.ERadaptive));
		}

		// Token: 0x06000CBD RID: 3261 RVA: 0x000372E4 File Offset: 0x000354E4
		public static IList<double> GenOMA(IList<double> data, int _length, double _speed, bool _ERadaptive)
		{
			int count = data.Count;
			_length = Math.Max(_length, 1);
			_speed = Math.Max(_speed, -1.5);
			double[] array = new double[count];
			double[,] stored = new double[count, 7];
			for (int i = 0; i < count; i++)
			{
				if (i > 1)
				{
					array[i] = OMA.iOMA(data[i], stored, (double)_length, _speed, _ERadaptive, i);
				}
				else
				{
					array[i] = data[i];
				}
			}
			return array;
		}

		// Token: 0x06000CBF RID: 3263 RVA: 0x0003741C File Offset: 0x0003561C
		private static double iOMA(double price, double[,] stored, double averagePeriod, double speed, bool adaptive, int barnum)
		{
			double num = stored[barnum - 1, 0];
			double num2 = stored[barnum - 1, 1];
			double num3 = stored[barnum - 1, 2];
			double num4 = stored[barnum - 1, 3];
			double num5 = stored[barnum - 1, 4];
			double num6 = stored[barnum - 1, 5];
			if (adaptive && averagePeriod > 1.0)
			{
				double num7 = averagePeriod / 2.0;
				double num8 = num7 * 5.0;
				int num9 = (int)Math.Ceiling(num8);
				num9 = Math.Min(barnum, num9);
				double num10 = Math.Abs(price - stored[barnum - num9, 6]);
				double num11 = 1E-11;
				for (int i = 1; i < num9; i++)
				{
					num11 += Math.Abs(price - stored[barnum - i, 6]);
				}
				averagePeriod = num10 / num11 * (num8 - num7) + num7;
				averagePeriod = Math.Min((double)barnum, averagePeriod);
			}
			double num12 = (2.0 + speed) / (1.0 + speed + averagePeriod);
			num += num12 * (price - num);
			num2 += num12 * (num - num2);
			double num13 = 1.5 * num - 0.5 * num2;
			num3 += num12 * (num13 - num3);
			num4 += num12 * (num3 - num4);
			double num14 = 1.5 * num3 - 0.5 * num4;
			num5 += num12 * (num14 - num5);
			num6 += num12 * (num5 - num6);
			double result = 1.5 * num5 - 0.5 * num6;
			stored[barnum, 0] = num;
			stored[barnum, 1] = num2;
			stored[barnum, 2] = num3;
			stored[barnum, 3] = num4;
			stored[barnum, 4] = num5;
			stored[barnum, 5] = num6;
			stored[barnum, 6] = price;
			return result;
		}

		// Token: 0x1700042B RID: 1067
		public IContext Context
		{
			// Token: 0x06000CC0 RID: 3264 RVA: 0x0003760E File Offset: 0x0003580E
			get;
			// Token: 0x06000CC1 RID: 3265 RVA: 0x00037616 File Offset: 0x00035816
			set;
		}

		// Token: 0x1700042A RID: 1066
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool ERadaptive
		{
			// Token: 0x06000CBB RID: 3259 RVA: 0x000372D2 File Offset: 0x000354D2
			get;
			// Token: 0x06000CBC RID: 3260 RVA: 0x000372DA File Offset: 0x000354DA
			set;
		}

		// Token: 0x17000428 RID: 1064
		[HandlerParameter(true, "15", Min = "1", Max = "50", Step = "1")]
		public int Length
		{
			// Token: 0x06000CB7 RID: 3255 RVA: 0x000372B0 File Offset: 0x000354B0
			get;
			// Token: 0x06000CB8 RID: 3256 RVA: 0x000372B8 File Offset: 0x000354B8
			set;
		}

		// Token: 0x17000429 RID: 1065
		[HandlerParameter(true, "0.5", Min = "-1.5", Max = "10", Step = "0.1", Name = "Speed:\n0.5-T3 Tilson\n2.5-T3 Fulks\\Matulich\n1-SMA\n2-LWMA\n7-Hull & Tema\n8-LSMA")]
		public double Speed
		{
			// Token: 0x06000CB9 RID: 3257 RVA: 0x000372C1 File Offset: 0x000354C1
			get;
			// Token: 0x06000CBA RID: 3258 RVA: 0x000372C9 File Offset: 0x000354C9
			set;
		}
	}
}
