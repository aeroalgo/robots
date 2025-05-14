using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000093 RID: 147
	[HandlerCategory("vvStoch"), HandlerName("StochasticCyberCycle")]
	public class StochCC : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000533 RID: 1331 RVA: 0x0001A58A File Offset: 0x0001878A
		public IList<double> Execute(IList<double> src)
		{
			return this.GenStochCC(src, this.Length, this.Alpha, this.Trigger, this.Context);
		}

		// Token: 0x06000532 RID: 1330 RVA: 0x0001A328 File Offset: 0x00018528
		public IList<double> GenStochCC(IList<double> src, int _Length, double alpha, bool trigger, IContext context)
		{
			int count = src.Count;
			int num = Math.Max(4, _Length);
			double[] array = new double[count];
			double[] array2 = new double[count];
			double[] array3 = new double[count];
			double[] array4 = new double[count];
			double[] array5 = new double[count];
			for (int i = num; i < count; i++)
			{
				array3[i] = (src[i] + 2.0 * src[i - 1] + 2.0 * src[i - 2] + src[i - 3]) / 6.0;
				array4[i] = (1.0 - 0.5 * alpha) * (1.0 - 0.5 * alpha) * (array3[i] - 2.0 * array3[i - 1] + array3[i - 2]) + 2.0 * (1.0 - alpha) * array4[i - 1] - (1.0 - alpha) * (1.0 - alpha) * array4[i - 2];
				if (i < 8)
				{
					array4[i] = (src[i] - 2.0 * src[i - 1] + src[i - 2]) / 4.0;
				}
				double num2 = array4[i];
				double num3 = array4[i];
				for (int j = 0; j < _Length; j++)
				{
					double val = array4[i - j];
					num2 = Math.Max(num2, val);
					num3 = Math.Min(num3, val);
				}
				array5[i] = 0.0;
				if (num2 != num3)
				{
					array5[i] = (array4[i] - num3) / (num2 - num3);
				}
				array[i] = (4.0 * array5[i] + 3.0 * array5[i - 1] + 2.0 * array5[i - 2] + array5[i - 3]) / 10.0;
				array[i] = 2.0 * (array[i] - 0.5);
				array2[i] = 0.96 * (array[i - 1] + 0.02);
			}
			if (!trigger)
			{
				return array;
			}
			return array2;
		}

		// Token: 0x170001C7 RID: 455
		[HandlerParameter(true, "0.07", Min = "0", Max = "1", Step = "0.01")]
		public double Alpha
		{
			// Token: 0x0600052E RID: 1326 RVA: 0x0001A305 File Offset: 0x00018505
			get;
			// Token: 0x0600052F RID: 1327 RVA: 0x0001A30D File Offset: 0x0001850D
			set;
		}

		// Token: 0x170001C9 RID: 457
		public IContext Context
		{
			// Token: 0x06000534 RID: 1332 RVA: 0x0001A5AB File Offset: 0x000187AB
			get;
			// Token: 0x06000535 RID: 1333 RVA: 0x0001A5B3 File Offset: 0x000187B3
			set;
		}

		// Token: 0x170001C6 RID: 454
		[HandlerParameter(true, "8", Min = "1", Max = "20", Step = "1")]
		public int Length
		{
			// Token: 0x0600052C RID: 1324 RVA: 0x0001A2F4 File Offset: 0x000184F4
			get;
			// Token: 0x0600052D RID: 1325 RVA: 0x0001A2FC File Offset: 0x000184FC
			set;
		}

		// Token: 0x170001C8 RID: 456
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool Trigger
		{
			// Token: 0x06000530 RID: 1328 RVA: 0x0001A316 File Offset: 0x00018516
			get;
			// Token: 0x06000531 RID: 1329 RVA: 0x0001A31E File Offset: 0x0001851E
			set;
		}
	}
}
