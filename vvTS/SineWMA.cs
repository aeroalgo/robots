using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200019A RID: 410
	[HandlerCategory("vvAverages"), HandlerName("SineWMA")]
	public class SineWMA : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000D00 RID: 3328 RVA: 0x000391F8 File Offset: 0x000373F8
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("SineWMA", new string[]
			{
				this.Period.ToString(),
				src.GetHashCode().ToString()
			}, () => SineWMA.GenSineWMA(src, this.Period));
		}

		// Token: 0x06000CFE RID: 3326 RVA: 0x000390F8 File Offset: 0x000372F8
		public static IList<double> GenSineWMA(IList<double> src, int period)
		{
			double[] array = new double[src.Count];
			for (int i = 0; i < src.Count; i++)
			{
				if (i < period)
				{
					array[i] = src[i];
				}
				else
				{
					array[i] = SineWMA.iSineWMA(src, period, i);
				}
			}
			return array;
		}

		// Token: 0x06000CFF RID: 3327 RVA: 0x00039140 File Offset: 0x00037340
		public static double iSineWMA(IList<double> price, int period, int barNum)
		{
			double num = 3.1415926535;
			double num2 = 0.0;
			double num3 = 0.0;
			for (int i = 0; i < period - 1; i++)
			{
				num3 += Math.Sin(num * (double)(i + 1) / (double)(period + 1));
				num2 += price[barNum - i] * Math.Sin(num * (double)(i + 1) / (double)(period + 1));
			}
			double result;
			if (num3 > 0.0)
			{
				result = num2 / num3;
			}
			else
			{
				result = 0.0;
			}
			return result;
		}

		// Token: 0x1700043D RID: 1085
		public IContext Context
		{
			// Token: 0x06000D01 RID: 3329 RVA: 0x00039264 File Offset: 0x00037464
			get;
			// Token: 0x06000D02 RID: 3330 RVA: 0x0003926C File Offset: 0x0003746C
			set;
		}

		// Token: 0x1700043C RID: 1084
		[HandlerParameter(true, "15", Min = "1", Max = "30", Step = "1")]
		public int Period
		{
			// Token: 0x06000CFC RID: 3324 RVA: 0x000390E7 File Offset: 0x000372E7
			get;
			// Token: 0x06000CFD RID: 3325 RVA: 0x000390EF File Offset: 0x000372EF
			set;
		}
	}
}
