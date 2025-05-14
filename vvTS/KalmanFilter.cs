using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200017F RID: 383
	[HandlerCategory("vvAverages"), HandlerName("Kalman filter")]
	public class KalmanFilter : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000C1C RID: 3100 RVA: 0x00034A34 File Offset: 0x00032C34
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("kalmanfilter", new string[]
			{
				this.K.ToString(),
				this.Sharpness.ToString(),
				src.GetHashCode().ToString()
			}, () => KalmanFilter.GenKF(src, this.K, this.Sharpness));
		}

		// Token: 0x06000C1B RID: 3099 RVA: 0x00034974 File Offset: 0x00032B74
		public static IList<double> GenKF(IList<double> src, double k, double sharpness)
		{
			int count = src.Count;
			double[] array = new double[count];
			double num = 0.0;
			double num2 = src[0];
			for (int i = 1; i < count; i++)
			{
				double num3 = src[i] - num2;
				double num4 = num2 + num3 * Math.Sqrt(sharpness * k / 100.0);
				num += num3 * k / 100.0;
				num2 = num4 + num;
				array[i] = num2;
			}
			return array;
		}

		// Token: 0x170003F9 RID: 1017
		public IContext Context
		{
			// Token: 0x06000C1D RID: 3101 RVA: 0x00034AB2 File Offset: 0x00032CB2
			get;
			// Token: 0x06000C1E RID: 3102 RVA: 0x00034ABA File Offset: 0x00032CBA
			set;
		}

		// Token: 0x170003F7 RID: 1015
		[HandlerParameter(true, "1", Min = "0.1", Max = "1", Step = "0.1")]
		public double K
		{
			// Token: 0x06000C17 RID: 3095 RVA: 0x0003494F File Offset: 0x00032B4F
			get;
			// Token: 0x06000C18 RID: 3096 RVA: 0x00034957 File Offset: 0x00032B57
			set;
		}

		// Token: 0x170003F8 RID: 1016
		[HandlerParameter(true, "1", Min = "0.1", Max = "1", Step = "0.1")]
		public double Sharpness
		{
			// Token: 0x06000C19 RID: 3097 RVA: 0x00034960 File Offset: 0x00032B60
			get;
			// Token: 0x06000C1A RID: 3098 RVA: 0x00034968 File Offset: 0x00032B68
			set;
		}
	}
}
