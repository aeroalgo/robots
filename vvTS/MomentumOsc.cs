using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200001F RID: 31
	[HandlerCategory("vvIndicators"), HandlerDecimals(2), HandlerName("CMO")]
	public class MomentumOsc : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs
	{
		// Token: 0x06000119 RID: 281 RVA: 0x000058F6 File Offset: 0x00003AF6
		public IList<double> Execute(IList<double> src)
		{
			return MomentumOsc.GenCMO(src, this.Period, this.Smooth);
		}

		// Token: 0x06000118 RID: 280 RVA: 0x000057E0 File Offset: 0x000039E0
		public static IList<double> GenCMO(IList<double> price, int periodcmo, int smooth = 0)
		{
			int count = price.Count;
			double[] array = new double[count];
			double[] array2 = new double[count];
			double[] array3 = new double[count];
			double num = 0.0;
			double num2 = 0.0;
			for (int i = 0; i < count; i++)
			{
				if (i < periodcmo)
				{
					array[i] = 0.0;
				}
				else
				{
					double num3 = price[i] - price[i - 1];
					array2[i] = ((num3 > 0.0) ? num3 : 0.0);
					array3[i] = ((num3 < 0.0) ? (-num3) : 0.0);
					num += array2[i];
					num2 += array3[i];
					if (i >= periodcmo)
					{
						num -= array2[i - periodcmo];
						num2 -= array3[i - periodcmo];
					}
					array[i] = (num - num2) / (num + num2) * 100.0;
				}
			}
			IList<double> result = array;
			if (smooth > 0)
			{
				result = LWMA.GenWMA(array, smooth);
			}
			return result;
		}

		// Token: 0x1700005C RID: 92
		[HandlerParameter(true, "15", Min = "1", Max = "30", Step = "1")]
		public int Period
		{
			// Token: 0x06000114 RID: 276 RVA: 0x000057BC File Offset: 0x000039BC
			get;
			// Token: 0x06000115 RID: 277 RVA: 0x000057C4 File Offset: 0x000039C4
			set;
		}

		// Token: 0x1700005D RID: 93
		[HandlerParameter(true, "0", Min = "1", Max = "20", Step = "1")]
		public int Smooth
		{
			// Token: 0x06000116 RID: 278 RVA: 0x000057CD File Offset: 0x000039CD
			get;
			// Token: 0x06000117 RID: 279 RVA: 0x000057D5 File Offset: 0x000039D5
			set;
		}
	}
}
