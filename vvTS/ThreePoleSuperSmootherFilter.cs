using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200019F RID: 415
	[HandlerCategory("vvAverages"), HandlerName("ThreePoleSuperSmootherFilter")]
	public class ThreePoleSuperSmootherFilter : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x06000D23 RID: 3363 RVA: 0x00039CAC File Offset: 0x00037EAC
		public IList<double> Execute(ISecurity sec)
		{
			return this.Context.GetData("threepolesmoother", new string[]
			{
				this.CutoffPeriod.ToString(),
				sec.get_CacheName()
			}, () => ThreePoleSuperSmootherFilter.Gen3PoleSSFilter(sec, this.CutoffPeriod));
		}

		// Token: 0x06000D22 RID: 3362 RVA: 0x00039B28 File Offset: 0x00037D28
		public static IList<double> Gen3PoleSSFilter(ISecurity src, int cutoffperiod)
		{
			double[] array = new double[src.get_Bars().Count];
			IList<double> lowPrices = src.get_LowPrices();
			IList<double> highPrices = src.get_HighPrices();
			double num = Math.Atan(1.0);
			double num2 = 45.0 / num;
			double num3 = 1.0 / num2;
			double num4 = Math.Atan(1.0) * 4.0;
			double num5 = Math.Exp(-num4 / (double)cutoffperiod);
			double num6 = 2.0 * num5 * Math.Cos(num3 * Math.Sqrt(3.0) * 180.0 / (double)cutoffperiod);
			double num7 = num5 * num5;
			double num8 = num6 + num7;
			double num9 = -(num7 + num6 * num7);
			double num10 = num7 * num7;
			double num11 = 1.0 - num8 - num9 - num10;
			for (int i = 0; i < src.get_Bars().Count; i++)
			{
				if (i < 4)
				{
					array[i] = (lowPrices[i] + highPrices[i]) / 2.0;
				}
				else
				{
					array[i] = num11 * ((lowPrices[i] + highPrices[i]) / 2.0) + num8 * array[i - 1] + num9 * array[i - 2] + num10 * array[i - 3];
				}
			}
			return array;
		}

		// Token: 0x17000447 RID: 1095
		public IContext Context
		{
			// Token: 0x06000D24 RID: 3364 RVA: 0x00039D10 File Offset: 0x00037F10
			get;
			// Token: 0x06000D25 RID: 3365 RVA: 0x00039D18 File Offset: 0x00037F18
			set;
		}

		// Token: 0x17000446 RID: 1094
		[HandlerParameter(true, "15", Min = "1", Max = "70", Step = "1")]
		public int CutoffPeriod
		{
			// Token: 0x06000D20 RID: 3360 RVA: 0x00039B15 File Offset: 0x00037D15
			get;
			// Token: 0x06000D21 RID: 3361 RVA: 0x00039B1D File Offset: 0x00037D1D
			set;
		}
	}
}
