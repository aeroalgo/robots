using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000052 RID: 82
	[HandlerCategory("vvIndicators"), HandlerName("Random Walk Index")]
	public class RWI : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x060002EB RID: 747 RVA: 0x0000E030 File Offset: 0x0000C230
		public IList<double> Execute(ISecurity sec)
		{
			return this.Context.GetData("RandomWalkIndex", new string[]
			{
				this.RWIperiod.ToString(),
				this.postSmooth.ToString(),
				this.Lows.ToString(),
				sec.get_CacheName()
			}, () => RWI.GenRWI(sec, this.Context, this.RWIperiod, this.postSmooth, this.Lows));
		}

		// Token: 0x060002EA RID: 746 RVA: 0x0000DD50 File Offset: 0x0000BF50
		public static IList<double> GenRWI(ISecurity sec, IContext ctx, int rwiperiod = 20, int postsmooth = 0, bool showlows = false)
		{
			IList<double> highPrices = sec.get_HighPrices();
			IList<double> lowPrices = sec.get_LowPrices();
			IList<double> closePrices = sec.get_ClosePrices();
			double[] RWIUp = new double[sec.get_Bars().Count];
			double[] RWIDown = new double[sec.get_Bars().Count];
			double[] array = new double[sec.get_Bars().Count];
			double[] array2 = new double[sec.get_Bars().Count];
			for (int i = rwiperiod; i < sec.get_Bars().Count; i++)
			{
				array2[i] = Math.Max(highPrices[i - 1], closePrices[i - 2]) - Math.Min(lowPrices[i - 1], closePrices[i - 2]);
				double num = 0.0;
				double num2 = 0.0;
				double num3 = array2[i];
				for (int j = 1; j < rwiperiod; j++)
				{
					num3 += array2[i - j];
					double num4 = num3 / ((double)j + 1.0) * Math.Sqrt((double)(j + 1));
					if (num4 != 0.0)
					{
						num = Math.Max(num, (highPrices[i] - lowPrices[i - j]) / num4);
						num2 = Math.Max(num2, (highPrices[i - j] - lowPrices[i]) / num4);
					}
				}
				RWIUp[i] = num;
				RWIDown[i] = num2;
				array[i] = 1.0;
			}
			IList<double> result = RWIUp;
			IList<double> result2 = RWIDown;
			if (postsmooth > 0)
			{
				result = ctx.GetData("JMA", new string[]
				{
					postsmooth.ToString(),
					0.ToString(),
					RWIUp.GetHashCode().ToString()
				}, () => JMA.GenJMA(RWIUp, postsmooth, 0));
				result2 = ctx.GetData("JMA", new string[]
				{
					postsmooth.ToString(),
					0.ToString(),
					RWIDown.GetHashCode().ToString()
				}, () => JMA.GenJMA(RWIDown, postsmooth, 0));
			}
			if (!showlows)
			{
				return result;
			}
			return result2;
		}

		// Token: 0x170000FD RID: 253
		public IContext Context
		{
			// Token: 0x060002EC RID: 748 RVA: 0x0000E0B7 File Offset: 0x0000C2B7
			get;
			// Token: 0x060002ED RID: 749 RVA: 0x0000E0BF File Offset: 0x0000C2BF
			set;
		}

		// Token: 0x170000FC RID: 252
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool Lows
		{
			// Token: 0x060002E8 RID: 744 RVA: 0x0000DD0F File Offset: 0x0000BF0F
			get;
			// Token: 0x060002E9 RID: 745 RVA: 0x0000DD17 File Offset: 0x0000BF17
			set;
		}

		// Token: 0x170000FB RID: 251
		[HandlerParameter(true, "0", Min = "3", Max = "20", Step = "1")]
		public int postSmooth
		{
			// Token: 0x060002E6 RID: 742 RVA: 0x0000DCFE File Offset: 0x0000BEFE
			get;
			// Token: 0x060002E7 RID: 743 RVA: 0x0000DD06 File Offset: 0x0000BF06
			set;
		}

		// Token: 0x170000FA RID: 250
		[HandlerParameter(true, "20", Min = "3", Max = "40", Step = "1")]
		public int RWIperiod
		{
			// Token: 0x060002E4 RID: 740 RVA: 0x0000DCED File Offset: 0x0000BEED
			get;
			// Token: 0x060002E5 RID: 741 RVA: 0x0000DCF5 File Offset: 0x0000BEF5
			set;
		}
	}
}
