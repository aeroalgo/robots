using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000135 RID: 309
	[HandlerCategory("vvRSI"), HandlerDecimals(2), HandlerName("LaguerreRSI")]
	public class LaguerreRSI : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x0600093B RID: 2363 RVA: 0x00026D21 File Offset: 0x00024F21
		public IList<double> Execute(IList<double> src)
		{
			return this.GenLaguerreRSI(src, this.Gamma, this.preSmooth, this.postSmooth);
		}

		// Token: 0x0600093A RID: 2362 RVA: 0x00026B58 File Offset: 0x00024D58
		public IList<double> GenLaguerreRSI(IList<double> src, double gamma, int presmooth, int postsmooth)
		{
			double[] array = new double[src.Count];
			double[] array2 = new double[src.Count];
			double[] array3 = new double[src.Count];
			double[] array4 = new double[src.Count];
			double[] array5 = new double[src.Count];
			double num = 0.0;
			IList<double> list = JMA.GenJMA(src, presmooth, 100);
			for (int i = 1; i < src.Count; i++)
			{
				array2[i] = (1.0 - gamma) * list[i] + gamma * array2[i - 1];
				array3[i] = -gamma * array2[i] + array2[i - 1] + gamma * array3[i - 1];
				array4[i] = -gamma * array3[i] + array3[i - 1] + gamma * array4[i - 1];
				array5[i] = -gamma * array4[i] + array4[i - 1] + gamma * array5[i - 1];
				double num2 = 0.0;
				double num3 = 0.0;
				if (array2[i] >= array3[i])
				{
					num2 = array2[i] - array3[i];
				}
				else
				{
					num3 = array3[i] - array2[i];
				}
				if (array3[i] >= array4[i])
				{
					num2 = num2 + array3[i] - array4[i];
				}
				else
				{
					num3 = num3 + array4[i] - array3[i];
				}
				if (array4[i] >= array5[i])
				{
					num2 = num2 + array4[i] - array5[i];
				}
				else
				{
					num3 = num3 + array5[i] - array4[i];
				}
				if (num2 + num3 != 0.0)
				{
					num = num2 / (num2 + num3);
				}
				array[i] = num * 100.0;
			}
			return JMA.GenJMA(array, postsmooth, 100);
		}

		// Token: 0x170002FC RID: 764
		public IContext Context
		{
			// Token: 0x0600093C RID: 2364 RVA: 0x00026D3C File Offset: 0x00024F3C
			get;
			// Token: 0x0600093D RID: 2365 RVA: 0x00026D44 File Offset: 0x00024F44
			set;
		}

		// Token: 0x170002F9 RID: 761
		[HandlerParameter(true, "0.7", Min = "0", Max = "1", Step = "0.1")]
		public double Gamma
		{
			// Token: 0x06000934 RID: 2356 RVA: 0x00026B24 File Offset: 0x00024D24
			get;
			// Token: 0x06000935 RID: 2357 RVA: 0x00026B2C File Offset: 0x00024D2C
			set;
		}

		// Token: 0x170002FB RID: 763
		[HandlerParameter(true, "1", Min = "0", Max = "20", Step = "1")]
		public int postSmooth
		{
			// Token: 0x06000938 RID: 2360 RVA: 0x00026B46 File Offset: 0x00024D46
			get;
			// Token: 0x06000939 RID: 2361 RVA: 0x00026B4E File Offset: 0x00024D4E
			set;
		}

		// Token: 0x170002FA RID: 762
		[HandlerParameter(true, "1", Min = "0", Max = "20", Step = "1")]
		public int preSmooth
		{
			// Token: 0x06000936 RID: 2358 RVA: 0x00026B35 File Offset: 0x00024D35
			get;
			// Token: 0x06000937 RID: 2359 RVA: 0x00026B3D File Offset: 0x00024D3D
			set;
		}
	}
}
